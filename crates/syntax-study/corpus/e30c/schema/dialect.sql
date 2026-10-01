-- The dialect's schema rule at DDL time (docs/study/E30c-design.md §3, clauses D1-D3).
-- Written after the corpus was committed (4d820f3). Installed once per database by the
-- cluster's superuser, because event triggers are superuser-only; the scripts under test run as
-- the provisioned non-superuser role.
--
-- Money-bearing: a column whose type is a composite type in schema `currency`, an array of one,
-- or a domain over one. Nothing else is money.

create schema if not exists dialect;
create schema if not exists currency;
create schema if not exists ledger;
create schema if not exists state;
create schema if not exists linear;
create schema if not exists capability;
grant usage on schema dialect to public;
grant usage, create on schema currency, ledger, state, linear, capability to public;

create function dialect.is_money(t oid) returns boolean language sql stable as $$
    with recursive walk(oid) as (
        select t
        union
        select coalesce(nullif(ty.typelem, 0), nullif(ty.typbasetype, 0))
          from pg_type ty join walk w on ty.oid = w.oid
         where coalesce(nullif(ty.typelem, 0), nullif(ty.typbasetype, 0)) is not null
    )
    select exists (
        select 1 from walk w join pg_type ty on ty.oid = w.oid
          join pg_namespace n on n.oid = ty.typnamespace
         where n.nspname = 'currency' and ty.typtype = 'c'
    )
$$;

create function dialect.money_columns(rel oid) returns setof name language sql stable as $$
    select a.attname from pg_attribute a
     where a.attrelid = rel and a.attnum > 0 and not a.attisdropped
       and dialect.is_money(a.atttypid)
     order by a.attnum
$$;

-- Conservation per (txn, currency) at commit: every money column sums to zero per txn.
create function dialect.conserve() returns trigger language plpgsql as $$
declare col name; bad bigint;
begin
    for col in select dialect.money_columns(tg_relid) loop
        execute format(
            'select txn from %I.%I where txn = $1 group by txn having coalesce(sum((%I).minor), 0) <> 0',
            tg_table_schema, tg_table_name, col)
          into bad using new.txn;
        if bad is not null then
            raise exception 'E30c D3: transaction % does not conserve in %.%.%',
                bad, tg_table_schema, tg_table_name, col;
        end if;
    end loop;
    return null;
end $$;

create function dialect.forbid_mutation() returns trigger language plpgsql as $$
begin
    raise exception 'E30c D3: %.% is a ledger and is append-only (attempted %)',
        tg_table_schema, tg_table_name, tg_op;
end $$;

create function dialect.on_ddl_end() returns event_trigger language plpgsql as $$
declare
    obj record; c record; nsp text; kind "char"; money boolean;
begin
    for obj in select * from pg_event_trigger_ddl_commands() loop
        if obj.classid <> 'pg_class'::regclass then
            continue;
        end if;
        select n.nspname, k.relkind into nsp, kind
          from pg_class k join pg_namespace n on n.oid = k.relnamespace
         where k.oid = obj.objid;
        -- Storage only: tables, partitioned tables, materialized views. A view stores nothing.
        if nsp is null or kind not in ('r', 'p', 'm') then
            continue;
        end if;
        money := exists (select 1 from dialect.money_columns(obj.objid));
        -- D1: money is stored only in ledger or state.
        if money and nsp not in ('ledger', 'state') then
            raise exception 'E30c D1: % holds money (%) and is in schema %, not ledger or state',
                obj.objid::regclass,
                (select string_agg(m, ', ') from dialect.money_columns(obj.objid) m), nsp;
        end if;
        if nsp = 'ledger' then
            -- D2: a ledger holds money and names its transaction.
            if not money then
                raise exception 'E30c D2: % is in schema ledger and holds no money column', obj.objid::regclass;
            end if;
            if not exists (select 1 from pg_attribute a where a.attrelid = obj.objid
                            and a.attname = 'txn' and not a.attisdropped) then
                raise exception 'E30c D2: % is in schema ledger and has no txn column', obj.objid::regclass;
            end if;
            if kind = 'm' then
                raise exception 'E30c D2: % is a materialized view in schema ledger; a ledger is a table',
                    obj.objid::regclass;
            end if;
            -- D3: the dialect's triggers, installed here and never disabled. A partition takes
            -- its parent's row triggers from PostgreSQL itself.
            if not exists (select 1 from pg_trigger t where t.tgrelid = obj.objid and t.tgname = 'dialect_conserve') then
                execute format('create constraint trigger dialect_conserve after insert on %s '
                               'deferrable initially deferred for each row execute function dialect.conserve()',
                               obj.objid::regclass);
            end if;
            if not exists (select 1 from pg_trigger t where t.tgrelid = obj.objid and t.tgname = 'dialect_append_only') then
                execute format('create trigger dialect_append_only before update or delete on %s '
                               'for each row execute function dialect.forbid_mutation()',
                               obj.objid::regclass);
            end if;
            if not exists (select 1 from pg_trigger t where t.tgrelid = obj.objid and t.tgname = 'dialect_no_truncate') then
                execute format('create trigger dialect_no_truncate before truncate on %s '
                               'for each statement execute function dialect.forbid_mutation()',
                               obj.objid::regclass);
            end if;
            for c in select t.tgname from pg_trigger t
                      where t.tgrelid = obj.objid and t.tgname like 'dialect\_%' and t.tgenabled = 'D' loop
                raise exception 'E30c D3: % on ledger table % may not be disabled', c.tgname, obj.objid::regclass;
            end loop;
        end if;
    end loop;
end $$;

-- D3, for drops: a dialect trigger may not be dropped while its table stands.
create function dialect.on_sql_drop() returns event_trigger language plpgsql as $$
declare d record; tables text[];
begin
    select array_agg(object_identity) into tables
      from pg_event_trigger_dropped_objects() where object_type = 'table';
    for d in select * from pg_event_trigger_dropped_objects()
              where object_type = 'trigger' and object_identity like 'dialect\_% on ledger.%' loop
        if tables is null or not (split_part(d.object_identity, ' on ', 2) = any (tables)) then
            raise exception 'E30c D3: % may not be dropped from a ledger table', d.object_identity;
        end if;
    end loop;
end $$;

create event trigger dialect_ddl_end on ddl_command_end execute function dialect.on_ddl_end();
create event trigger dialect_sql_drop on sql_drop execute function dialect.on_sql_drop();

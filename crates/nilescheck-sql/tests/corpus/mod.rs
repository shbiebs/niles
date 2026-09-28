//! The statement corpus, shared by the parser's tests and by the test that holds PostgreSQL
//! to the same verdicts (`postgres_agrees.rs`): what this parser accepts, PostgreSQL's parser
//! must accept, and what it refuses, PostgreSQL must refuse with a syntax error.
#![allow(dead_code)]

pub const QUERIES: &[&str] = &[
        "select 1",
        "select from t",
        "select a, b as c, d e from t where a = 1 and not b or c is not null",
        "select distinct on (a) a, b from t order by a, b desc nulls last",
        "select * from a join b using (id) left join c on c.id = a.id cross join d natural join e",
        "select t.* , (t.x).y, arr[1], arr[1:2], arr[:3] from t",
        "select count(*), sum(x) filter (where x > 0), string_agg(s, ',' order by s) from t group by rollup(a, b), cube(c), grouping sets ((a), ())",
        "select percentile_cont(0.5) within group (order by x) from t",
        "select row_number() over (partition by a order by b rows between unbounded preceding and current row) from t window w as (partition by a)",
        "select sum(x) over w from t window w as (order by y)",
        "with recursive r(n) as (select 1 union all select n + 1 from r where n < 10) select n from r",
        "with x as materialized (select 1), y as not materialized (select 2) select * from x, y",
        "select a from t union select b from u intersect select c from v except all select d from w",
        "(select 1) union all (select 2) order by 1 limit 1 offset 0",
        "select * from t order by a fetch first 3 rows with ties",
        "select * from t for update of t skip locked",
        "values (1, 'a'), (2, 'b') order by 1",
        "table t",
        "select case when a then 1 when b then 2 else 3 end, case x when 1 then 'a' end from t",
        "select cast(a as numeric(18, 2)), a::text, a::varchar(10)[], '1'::interval, b::double precision from t",
        "select date '2026-01-01', interval '1 day', timestamp with time zone '2026-01-01 00:00Z' , numeric '1.5'",
        "select x between 1 and 2, x not between symmetric 2 and 1, x in (1, 2), x not in (select y from u), x = any(array[1,2]), x <> all (select 1)",
        "select a like 'x%' escape '!', a not ilike 'y', a similar to 'z', a is distinct from b, a is not distinct from b",
        "select exists (select 1 from t), (select max(x) from t), array(select 1), row(1, 2), (1, 2)",
        "select extract(epoch from now()), position('a' in 'cat'), substring('abc' from 2 for 1), trim(both ' ' from x), overlay('abc' placing 'z' from 2)",
        "select current_date, current_timestamp, current_timestamp(3), localtime, current_user, session_user",
        "select a || b, a @> b, a -> 'k', a ->> 'k', a #>> '{a,b}', -a, @a, |/ 16, 2 ^ 3 ^ 2, a operator(pg_catalog.+) b",
        "select x at time zone 'UTC', x collate \"C\" from t",
        "select f(a => 1, b := 2), g(variadic array[1,2]) from t",
        "select * from generate_series(1, 10) with ordinality as g(n, i), lateral (select n) s, unnest(array[1]) u",
        "select * from t tablesample system (10) where true",
        "select $1::int + $2",
];

/// Refused by both parsers (adjacent string constants on one line are a syntax error).
pub const ADJACENT: &str = "select 'a' 'b'";

pub const DML: &[&str] = &[
        "insert into t values (1, 2), (3, default)",
        "insert into t (a, b) select a, b from u on conflict (a) do update set b = excluded.b where t.b <> excluded.b returning *",
        "insert into t default values returning id",
        "insert into t as x (a) values (1) on conflict on constraint t_pkey do nothing",
        "with d as (delete from t where a = 1 returning *) insert into u select * from d",
        "update t set a = 1, (b, c) = (2, 3), d = default from u where t.id = u.id returning t.a",
        "update only t as x set a = (select 1) where current of c",
        "delete from t using u where t.id = u.id returning *",
        "delete from only t x where x.a in (1, 2)",
];

pub const DDL: &[&str] = &[
        "truncate table postings restart identity cascade",
        "merge into t using u on t.id = u.id when matched and u.x > 0 then update set x = u.x when matched then delete when not matched then insert (id, x) values (u.id, u.x)",
        "with s as (select 1 as id) merge into t using s on t.id = s.id when not matched then do nothing",
        "call p(1, 'x')",
        "explain (analyze, buffers) select * from t",
        "explain select 1",
        "grant select, update (a, b) on table t, u to bench with grant option",
        "grant all privileges on all tables in schema public to bench",
        "revoke execute on function f(int) from public",
        "grant pg_read_all_settings to bench",
        "create sequence if not exists s increment by 2 start with 10 owned by t.id",
        "create rule no_update as on update to t do instead nothing",
        "create or replace rule r as on insert to t where new.a > 0 do also (insert into u values (new.a); update v set n = n + 1)",
        "create operator === (leftarg = int4, rightarg = int4, function = int4eq, commutator = ===)",
        "create aggregate mysum(int) (sfunc = int4pl, stype = int, initcond = '0')",
        "create cast (int as text) with function f(int) as implicit",
        "create extension if not exists pgcrypto with schema public",
        "create policy p on t for select to bench using (acct = 1)",
        "create role bench2 login password null",
        "create collation c1 (provider = icu, locale = 'und')",
        "create text search dictionary d (template = simple, stopwords = english)",
        "create table t (id bigserial primary key, amt bigint not null default 0 check (amt >= 0), cur smallint references c (id) on delete cascade, note text collate \"C\", ts timestamptz default now(), g int generated always as (id * 2) stored, constraint u unique (cur, id))",
        "create temp table if not exists t (a int) on commit drop",
        "create unlogged table t as select 1 as a with no data",
        "create table p (a int, b text) partition by range (a)",
        "create or replace view v (a, b) with (security_barrier = true, check_option = local) as select a, b from t with local check option",
        "create materialized view mv as select a, sum(b)::bigint as s from t group by a with no data",
        "create unique index concurrently if not exists i on t using btree (a desc nulls last, lower(b)) include (c) where a > 0",
        "create type money_usd as (amount numeric(38, 2), currency char(3))",
        "create type mood as enum ('sad', 'ok', 'happy')",
        "create domain posamt as numeric(18, 2) not null check (value > 0)",
        "create schema if not exists ledger",
        "comment on column t.amt is 'minor units'",
        "comment on view v is null",
        "drop view if exists v, w cascade",
        "drop trigger x on t",
        "drop function f(int, text)",
        "alter table t disable trigger postings_immutable",
        "alter table t add column x int",
        "create constraint trigger c after insert on t deferrable initially deferred for each row execute function f()",
        "create trigger b before update or delete on t for each row when (old.a is distinct from new.a) execute procedure g()",
        "grant select on t to bench",
        "set search_path = arm",
        "begin; commit; rollback",
        "create function f(a int, b text default 'x', out c int) returns int language sql immutable strict as $$ select a $$",
        "create function f2(int) returns setof record language sql stable parallel safe cost 10 as 'select $1'",
        "create function f3() returns table (a int, b text) language sql return (select 1)",
        "create procedure p(inout x int) language plpgsql as $$ begin x := x + 1; end $$",
        "create function f4(x int) returns int language sql begin atomic select x + 1; end",
];

pub const REFUSED: &[&str] = &[
    "merge into t using u on t.id = u.id",
    "merge into t using u on true when matched then insert values (1)",
    "create rule r as on truncate to t do nothing",
    "grant select on t",
    "select * from t fetch first 3 rows with ties",
    "select from where",
    "select a from t where",
    "select case end",
    "insert t values (1)",
    "create table t (a int",
    "select 1 < 2 < 3",
];

pub const PLPGSQL: &str = r#"
create table postings (id bigserial primary key, acct bigint not null, amt numeric not null);
create or replace function transfer(p_from bigint, p_to bigint, p_amt bigint)
returns void language plpgsql as $body$
<<outer>>
declare
    v_bal numeric := 0;
    k constant int = 3;
    r record;
    c cursor for select * from postings;
    v_id postings.id%type;
begin
    -- @linear hold
    select sum(amt) into strict v_bal from postings where acct = p_from;
    if v_bal < p_amt then
        raise exception 'insufficient funds: % < %', v_bal, p_amt using errcode = 'P0001';
    elsif v_bal = p_amt then
        raise notice 'exact';
    else
        null;
    end if;
    insert into postings (acct, amt) values (p_from, -p_amt) returning id into v_id;
    insert into postings (acct, amt) values (p_to, p_amt);
    perform pg_notify('x', 'y');
    for i in 1..10 by 2 loop
        continue when i = 3;
        exit outer when i > 8;
    end loop;
    for i in reverse 10..1 loop null; end loop;
    for r in select * from postings where acct = p_to loop
        v_bal := v_bal + r.amt;
    end loop;
    foreach k in array array[1,2] loop null; end loop;
    <<l>> while v_bal > 0 loop v_bal := v_bal - 1; end loop l;
    case k when 1, 2 then null; else null; end case;
    execute format('select %L', p_amt) into v_bal using p_amt;
    get diagnostics k = row_count;
    assert v_bal >= 0, 'negative';
    begin
        update postings set amt = amt where false;
    exception when unique_violation or sqlstate '23505' then
        raise;
    end;
    return;
end outer;
$body$;
"#;

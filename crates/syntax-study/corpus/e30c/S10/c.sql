-- S10 correct: disabling the author's own trigger on a state table.
create domain linear.ticket as bigint;
create table state.tickets (id linear.ticket primary key, amt currency.usd not null);
create function state.touch() returns trigger language plpgsql as $$ begin return new; end $$;
create trigger touch before insert on state.tickets for each row execute function state.touch();
alter table state.tickets disable trigger touch;

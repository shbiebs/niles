-- S09 correct: a trigger the author made on a state table is the author's to drop.
create domain linear.ticket as bigint;
create table state.tickets (id linear.ticket primary key, amt currency.usd not null);
create function state.touch() returns trigger language plpgsql as $$ begin return new; end $$;
create trigger touch before insert on state.tickets for each row execute function state.touch();
drop trigger touch on state.tickets;

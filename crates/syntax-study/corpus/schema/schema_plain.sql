-- E30's schema for PRQL: the same four relations with plain columns — a currency as text, an
-- amount as bigint minor units, a value date as a date. PRQL queries relations and compiles to
-- SQL; the typed money of SQL+C+L is that surface's discipline, not the relational model's,
-- so PRQL is given the relations a PRQL user would have.
create table parties (id bigint primary key, parent bigint);
create table accounts (id bigint primary key, owner bigint not null, desk bigint not null);
create table postings (txn bigint not null, acct bigint not null, cur text not null,
    amt bigint not null, epoch bigint not null, value_date date not null);
create table holds (id bigint primary key, acct bigint not null, cur text not null,
    amount bigint not null, open boolean not null);

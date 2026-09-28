-- D3: half a yen. jpy has scale 0; the minor-unit literal 100.5 would be rounded into the bigint.
create function f() returns jpy language sql as $$ select row(100.5)::jpy $$;

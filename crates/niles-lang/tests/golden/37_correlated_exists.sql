select * from t where exists (select 1 from u where u.k = t.k)

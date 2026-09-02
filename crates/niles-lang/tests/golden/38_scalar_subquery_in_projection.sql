select k, (select max(w) from u where u.k = t.k) from t

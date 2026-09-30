select k, sum(v) from t group by k having count(v) > 1

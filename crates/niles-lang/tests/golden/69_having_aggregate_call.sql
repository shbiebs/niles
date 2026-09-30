select k, sum(v) from t group by k having sum(v) > 20

select k, row_number() over (partition by k order by v) as rn from t

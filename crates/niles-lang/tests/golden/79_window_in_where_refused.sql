select k from t where rank() over (order by k) = 1

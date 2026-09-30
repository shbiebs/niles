select acct, cur, sum(x)
from (
    select acct, cur, amt as x from postings
    union all
    select acct, cur, -amount from holds where open
) flows
group by acct, cur

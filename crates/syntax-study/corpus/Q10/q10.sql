select acct, cur, sum(x)
from (
    select acct, cur, coalesce((amt_usd).minor, (amt_eur).minor, (amt_jpy).minor) as x from postings
    union all
    select acct, cur, -coalesce((amt_usd).minor, (amt_eur).minor) from holds where open
) flows
group by acct, cur

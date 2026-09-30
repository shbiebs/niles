select acct, desk, rank() over (partition by desk order by balance desc)
from (
    select p.acct, a.desk, (sum(p.amt_usd)).minor as balance
    from postings p
    join accounts a on a.id = p.acct
    where p.cur = 'usd'
    group by p.acct, a.desk
) b

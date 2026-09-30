select txn, value_date, cur, coalesce((amt_usd).minor, (amt_eur).minor, (amt_jpy).minor)
from postings
where acct = 3
  and value_date between date '2026-01-11' and date '2026-01-31'
  and epoch <= 5

select txn, value_date, cur, amt
from postings
where acct = 3
  and value_date between date '2026-01-11' and date '2026-01-31'
  and epoch <= 5

(cd migrations && ./sqitch rebase -y --target local)
PGPASSWORD=password psql -f ./migrations/seed.sql -h localhost -U admin -d monk
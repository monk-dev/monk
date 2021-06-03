docker-compose -f docker/docker-compose.yml build
docker-compose -f docker/docker-compose.yml up -d db

sleep 1

(cd migrations && ./sqitch deploy --target local)

PGPASSWORD=password psql -f ./migrations/seed.sql -h localhost -U admin -d monk
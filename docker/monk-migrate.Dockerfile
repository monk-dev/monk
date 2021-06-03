FROM sqitch/sqitch:latest

COPY migrations /migrations

WORKDIR /migrations
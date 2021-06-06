const http = require("http");
const { postgraphile } = require("postgraphile");

const PgSimplifyInflectorPlugin = require("@graphile-contrib/pg-simplify-inflector");

console.log(process.env.DATABASE_URL);

// "npx postgraphile -c postgres://admin:password@localhost:5432/monk \ 
// --watch --enhance - graphiql
// --append - plugins @graphile-contrib / pg - simplify - inflector \
// --simple - collections only"

http
    .createServer(
        postgraphile(process.env.DATABASE_URL, "public", {
            appendPlugins: [PgSimplifyInflectorPlugin],

            graphileBuildOptions: {
                pgOmitListSuffix: true
            },
            enableCors: true,
            watchPg: true,
            graphiql: true,
            enhanceGraphiql: true,
            retryOnInitFail: true,
        })
    )
    .listen(process.env.PORT);

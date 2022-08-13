const fs = require("fs");
const process = require("process");

const total = process.argv
  .slice(2)
  .flatMap((file) => JSON.parse(fs.readFileSync(file, "utf-8"))["users"]);

const lookup = total.reduce((a, e) => {
  a[e.accountName] = ++a[e.accountName] || 0;
  return a;
}, {});

console.log(total.filter((e) => lookup[e.accountName]));

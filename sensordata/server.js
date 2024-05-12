import { $ } from "bun";

Bun.serve({
  port: 8080,
  error: (req, error) => {
    console.log(req);
    console.error(error);
    process.exit(1);
  },
  async fetch(req) {
    const output =
      await $`frida -q -U "MyMacca's" --runtime=v8 -l getsensordata.js`.quiet();

    console.log(`${req.method} ${req.url}: ${output.exitCode}`);

    return Response(
      JSON.stringify({ sensor_data: output.text().replace("\n", "") }),
      {
        status: 200,
      },
    );
  },
});

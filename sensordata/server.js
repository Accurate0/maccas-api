import { $ } from "bun";

Bun.serve({
  port: 8080,
  async fetch(_req) {
    const output =
      await $`frida -q -U "MyMacca's" --runtime=v8 -l getsensordata.js`;

    return Response(
      JSON.stringify({ sensor_data: output.text().replace("\n", "") }),
      {
        status: 200,
      },
    );
  },
});

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "standalone",
  webpack(config, options) {
    config.module.rules.push({
      test: /\.(graphql|gql)$/,
      use: [require.resolve("graphql-tag/loader")],
    });

    return config;
  },
  images: {
    remotePatterns: [
      {
        protocol: "https",
        hostname: "i.maccas.one",
      },
    ],
  },
};

module.exports = nextConfig;

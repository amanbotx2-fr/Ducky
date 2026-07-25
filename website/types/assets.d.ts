/// <reference types="@cloudflare/workers-types" />

declare module "*.png" {
  const image: import("next/image").StaticImageData;
  export default image;
}

declare namespace Cloudflare {
  interface Env {
    DB?: D1Database;
  }
}

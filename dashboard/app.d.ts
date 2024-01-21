/// <reference types="lucia" />
declare namespace Lucia {
  type Auth = import("./src/db").Auth;
  type DatabaseUserAttributes = {
    username: string;
  };
  type DatabaseSessionAttributes = {
    access_token: string;
  };
}

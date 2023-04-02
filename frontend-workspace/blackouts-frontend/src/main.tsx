import React from "react";
import ReactDOM from "react-dom/client";
import { Auth0Provider } from "@auth0/auth0-react";
import { SWRConfig } from "swr";

import App from "./App";
import Theme from "./theme";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Theme>
      <SWRConfig
        value={{
          fetcher: async (resource, init) => {
            const apiURL = `/api${resource}`;
            const res = await fetch(apiURL, init);
            return await res.json();
          },
        }}
      >
        <Auth0Provider
          domain="blackouts-development.eu.auth0.com"
          clientId="AvpZjg5KXcdiZip56F2tYt69lM1GiABm"
          authorizationParams={{
            redirect_uri: `${window.location.origin}/authenticated`,
            audience: "https://blackouts.co.ke",
          }}
        >
          <App />
        </Auth0Provider>
      </SWRConfig>
    </Theme>
  </React.StrictMode>
);

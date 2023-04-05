import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App";
import Theme from "./theme";
import AuthProv from "./providers/Auth";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Theme>
      <AuthProv>
        <App />
      </AuthProv>
    </Theme>
  </React.StrictMode>
);

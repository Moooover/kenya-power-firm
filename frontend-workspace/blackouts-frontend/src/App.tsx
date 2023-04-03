import "./App.css";
import { useAuth0 } from "@auth0/auth0-react";
import { useEffect } from "react";
import Login from "./components/Login";
import Layout from "./components/Layout";
import { RouterProvider } from "react-router-dom";
import { router } from "./routes";
import { SWRConfig } from "swr";
import { useAuth } from "./providers/Auth";

function App() {
  const { isAuthenticated, token } = useAuth();
  console.log(token);
  return (
    <div>
      <div />
      <Layout>
        <SWRConfig
          value={{
            fetcher: async (resource, init) => {
              const apiURL = `/api${resource}`;
              const res = await fetch(apiURL, init);
              return await res.json();
            },
          }}
        ></SWRConfig>
        <div>
          {!isAuthenticated ? <Login /> : <RouterProvider router={router} />}
        </div>
      </Layout>
    </div>
  );
}

export default App;

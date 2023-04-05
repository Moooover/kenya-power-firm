import Login from "./components/Login";
import { RouterProvider } from "react-router-dom";
import { router } from "./routes";
import { SWRConfig } from "swr";
import { useAuth } from "./providers/Auth";
import axios, { CreateAxiosDefaults } from "axios";

const baseURL: string = import.meta.env.VITE_API_URL;

const config: CreateAxiosDefaults<any> | undefined = import.meta.env.PROD
  ? {
      baseURL,
    }
  : undefined;

const instance = axios.create(config);
instance.defaults.headers.common["Content-Type"] = "application/json";

function App() {
  const { isAuthenticated, token } = useAuth();
  return (
    <div>
      <div />
      {token && isAuthenticated ? (
        <SWRConfig
          value={{
            fetcher: async (resource, init) => {
              instance.defaults.headers.common[
                "Authorization"
              ] = `Bearer ${token}`;
              const apiURL = `/api${resource}`;
              const res = await instance(apiURL);
              return await res.data;
            },
          }}
        >
          <RouterProvider router={router} />
        </SWRConfig>
      ) : (
        <Login />
      )}
    </div>
  );
}

export default App;

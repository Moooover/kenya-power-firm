import Login from "./components/LoginContainer/Login";
import { RouterProvider } from "react-router-dom";
import { router } from "./routes";
import { SWRConfig } from "swr";
import { useAuth } from "./providers/Auth";
import { instance } from "./axios";

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
              const res = await instance(resource);
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

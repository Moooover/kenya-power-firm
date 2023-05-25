import Login from "./components/LoginContainer/Login";
import { RouterProvider } from "react-router-dom";
import { router } from "./routes";
import React from "react";
import { SWRConfig } from "swr";
import { useAuth } from "./providers/Auth";
import { instance } from "./axios";
import SnackBar from "./components/SnackBar";

function App() {
  const { isAuthenticated, token } = useAuth();

  const [isErrorSnackBarOpen, setIsErrorSnackBarOpen] = React.useState(false);

  const closeErrorSnackBar = () => {
    setIsErrorSnackBarOpen(false);
  };
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
            onError: (_err) => {
              setIsErrorSnackBarOpen(true);
            },
          }}
        >
          <RouterProvider router={router} />
          <SnackBar
            open={isErrorSnackBarOpen}
            content={"Something went wrong. Please try again."}
            close={closeErrorSnackBar}
          />
        </SWRConfig>
      ) : (
        <Login />
      )}
    </div>
  );
}

export default App;

import "./App.css";
import { useAuth0 } from "@auth0/auth0-react";
import { useEffect } from "react";
import Login from "./components/Login";
import Layout from "./components/Layout";
import { RouterProvider } from "react-router-dom";
import { router } from "./routes";

function App() {
  const { getAccessTokenSilently, isAuthenticated } = useAuth0();

  useEffect(() => {
    getAccessTokenSilently().then(console.log);
  }, [getAccessTokenSilently]);
  return (
    <div>
      <div />
      <Layout>
        <div>
          {!isAuthenticated ? <Login /> : <RouterProvider router={router} />}
        </div>
      </Layout>
    </div>
  );
}

export default App;

import React, { createContext, useContext, useEffect, useState } from "react";
import { Auth0Provider, useAuth0 } from "@auth0/auth0-react";
import useSWRMutation from "swr/mutation";

async function authenticate(
  url: string,
  {
    arg: { name, email, token },
  }: { arg: { name: string; email: string; token: string } }
) {
  return fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ name, email }),
  });
}

type AuthProps = {
  token: string | null;
  isAuthenticated: boolean;
};
const defaultValues: AuthProps = {
  token: null,
  isAuthenticated: false,
};
export const AuthContext = createContext(defaultValues);

type Props = {};

export default function AuthProv(props: React.PropsWithChildren<Props>) {
  return (
    <Auth0Provider
      domain="blackouts-development.eu.auth0.com"
      clientId="AvpZjg5KXcdiZip56F2tYt69lM1GiABm"
      authorizationParams={{
        redirect_uri: `${window.location.origin}/authenticated`,
        audience: "https://blackouts.co.ke",
      }}
    >
      <AuthConsumer>{props.children}</AuthConsumer>
    </Auth0Provider>
  );
}

function AuthConsumer(props: React.PropsWithChildren<Props>) {
  const { getAccessTokenSilently, isAuthenticated, user } = useAuth0();
  const [token, setToken] = useState<string | null>(null);
  const [isAuth, setIsAuth] = React.useState(false);
  const { trigger } = useSWRMutation("/api/authenticate", authenticate);
  const details =
    user?.name && user.email ? { name: user.name, email: user.email } : null;
  console.log("user details ", user);
  useEffect(() => {
    async function authenticate() {
      if (isAuthenticated && details) {
        try {
          const token = await getAccessTokenSilently();
          const formattedToken = token.replace(/(\r\n|\n|\r)/gm, "");
          const _ = await trigger({ ...details, token: formattedToken });
          setToken(formattedToken);
          setIsAuth(true);
        } catch (e) {
          console.log(e);
        }
      }
    }
    authenticate();
  }, [isAuthenticated, details]);
  return (
    <AuthContext.Provider value={{ token, isAuthenticated: isAuth }}>
      {props.children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  return useContext(AuthContext);
}

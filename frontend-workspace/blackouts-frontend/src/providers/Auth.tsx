import React, { createContext, useContext, useEffect, useState } from "react";
import { Auth0Provider, useAuth0 } from "@auth0/auth0-react";
import useSWRMutation from "swr/mutation";
import { instance } from "../axios";

async function authenticate(
  url: string,
  {
    arg: { name, email, token },
  }: { arg: { name: string; email: string; token: string } }
) {
  return instance.post(
    url,
    { name, email },
    {
      headers: {
        Authorization: `Bearer ${token}`,
      },
    }
  );
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

const domain = import.meta.env.VITE_AUTH_DOMAIN;
const clientId = import.meta.env.VITE_AUTH_CLIENT_ID;
const audience = import.meta.env.VITE_AUTH_AUDIENCE;

export default function AuthProv(props: React.PropsWithChildren<Props>) {
  return (
    <Auth0Provider
      domain={domain}
      clientId={clientId}
      authorizationParams={{
        redirect_uri: `${window.location.origin}/`,
        audience,
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
  const { trigger } = useSWRMutation("/authenticate", authenticate);
  const details =
    user?.name && user.email ? { name: user.name, email: user.email } : null;
  useEffect(() => {
    async function authenticate() {
      if (isAuthenticated && details) {
        try {
          const token = await getAccessTokenSilently();
          const _ = await trigger({ ...details, token });
          setToken(token);
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

export function useToken() {
  const { token } = useAuth();
  if (token == null) {
    throw Error("Token not found");
  }

  return token;
}

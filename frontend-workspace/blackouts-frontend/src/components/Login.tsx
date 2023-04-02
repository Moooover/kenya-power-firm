import React from "react";
import { useAuth0 } from "@auth0/auth0-react";
import LoginTwoToneIcon from "@mui/icons-material/LoginTwoTone";
import Button from "@mui/material/Button";
console.log(import.meta.env);

const Login = () => {
  const { loginWithRedirect } = useAuth0();
  return (
    <div>
      <Button
        size="large"
        endIcon={<LoginTwoToneIcon />}
        onClick={() => loginWithRedirect()}
      >
        Login
      </Button>
    </div>
  );
};

export default Login;

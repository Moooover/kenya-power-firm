import './App.css'
import LoginButton from "./components/Login";
import {useAuth0} from "@auth0/auth0-react";
import {useEffect} from "react";

function App() {
        const { getAccessTokenSilently, loginWithRedirect } = useAuth0();

        useEffect(() => {
            getAccessTokenSilently().then(console.log)
        }, [getAccessTokenSilently])
  return (
    <div className="App">
     <LoginButton />
    </div>
  )
}

export default App

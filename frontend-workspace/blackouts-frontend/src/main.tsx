import React from 'react'
import ReactDOM from 'react-dom/client'
import { Auth0Provider } from "@auth0/auth0-react";

import App from './App'
import './index.css'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
     <Auth0Provider
        domain="blackouts-development.eu.auth0.com"
        clientId="AvpZjg5KXcdiZip56F2tYt69lM1GiABm"
        authorizationParams={{
          redirect_uri: window.location.origin
        }}
      >
        <App />
     </Auth0Provider>
  </React.StrictMode>,
)

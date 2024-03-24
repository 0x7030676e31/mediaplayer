import { render } from "solid-js/web";
import { lazy } from "solid-js";
import { Router, Route } from "@solidjs/router";

import App from "./App";
import "./index.scss";

const Library = lazy(() => import("./components/library"));
const Home = lazy(() => import("./components/home"));
const Logs = lazy(() => import("./components/logs"));

render(
  () => (
    <Router root={App}>
      <Route path="/" component={Home} />
      <Route path="/library" component={Library} />
      <Route path="/logs" component={Logs} />
    </Router>
  ),
  document.getElementById("root")!
);

import { render } from "solid-js/web";
import { lazy } from "solid-js";
import { Router, Route } from "@solidjs/router";

import App from "./App";
import "./index.scss";

const Library = lazy(() => import("./components/library"));
const Home = lazy(() => import("./components/home"));
const Logs = lazy(() => import("./components/logs"));
// const Groups = lazy(() => import("./components/groups"))

render(
  () => (
    <Router root={App}>
      <Route path="/mp" component={Home} />
      <Route path="/mp/library" component={Library} />
      <Route path="/mp/logs" component={Logs} />
      {/* <Route path="/mp/groups" component={Groups} /> */}
    </Router>
  ),
  document.getElementById("root")!
);

import React, { Component } from "react";
import { BrowserRouter as Router, Switch, Route } from "react-router-dom";

import { Layout, Breadcrumb } from 'antd';

const { Content, Footer } = Layout;

import Home from './pages/Home';
import Job from './pages/Job.jsx';
import Login from './pages/Login.jsx';
import Project from './pages/Project.jsx';
import Projects from './pages/Projects.jsx';
import Tokens from './pages/Tokens.jsx';
import TopMenu from './components/TopMenu.jsx'
import Triggers from './pages/Triggers.jsx';
import Worker from './pages/Worker.jsx';
import Workers from './pages/Workers.jsx';

class App extends Component {
  render() {
    return (
      <Router>
        <Layout>
          <TopMenu />

          <Switch>
            <Route path="/projects/:id" component={Project} />
            <Route path="/projects" component={Projects} />
            <Route path="/jobs/:id/tokens/:trigger_datetime" component={Tokens} />
            <Route path="/jobs/:job_id/triggers/:trigger_id" component={Triggers} />
            <Route path="/jobs/:id/:tab" component={Job} />
            <Route path="/jobs/:id" component={Job} />
            <Route path="/workers/:id" component={Worker} />
            <Route path="/workers" component={Workers} />
            <Route path="/login" component={Login} />
            <Route path="/" component={Home} />
          </Switch>

          <Footer style={{ textAlign: 'center' }}>
            Waterwheel - {VERSION}
          </Footer>
        </Layout>
      </Router>
    );
  }
}

export default App;


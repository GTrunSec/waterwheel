import React, { Component, Fragment } from "react";
import { Link } from "react-router-dom";
import { Table, Layout, Breadcrumb } from 'antd';
import styled from 'styled-components'
import axios from 'axios';

const { Content } = Layout;


function makeColumns() {
    return [
        {
            title: 'Id',
            dataIndex: 'uuid',
            key: 'uuid',
            render: (text, record) => (
                <Link to={`/workers/${record.uuid}`}>
                    {text}
                </Link>
            ),
        },{
            title: 'UI Address',
            dataIndex: 'addr',
            key: 'addr',
            render: text => <a href={`http://${text}`}>{text}</a>,
        },{
            title: 'Last Seen',
            dataIndex: 'last_seen_datetime',
            key: 'last_seen_datetime',
        }
    ];
}


const Body = styled.div`
    padding: 24px;
    background: #fff;
`;


class Workers extends Component {
    constructor(props) {
        super(props);

        this.columns = makeColumns();

        this.state = {
            loading: false,
            workers: []
        };
    }

    async fetchWorkers() {
        try {
            this.setState({
                loading: true
            });
            let resp = await axios.get('/api/workers');
            this.setState({
                loading: false,
                workers: resp.data
            });
        } catch(e) {
            console.log(e);
            this.setState({
                loading: false,
                workers:[]
            });
        }
    }

    componentDidMount() {
        this.fetchWorkers()
    }

    render() {
        const { workers, loading } = this.state;

        return (
            <Layout>
                <Content style={{padding: '50px'}}>
                    <Breadcrumb style={{paddingBottom: '12px'}}>
                        <Breadcrumb.Item><Link to="/">Home</Link></Breadcrumb.Item>
                        <Breadcrumb.Item><Link to="/workers">Workers</Link></Breadcrumb.Item>
                    </Breadcrumb>
                    <Body>
                        <Table columns={this.columns} dataSource={workers} loading={loading}/>
                    </Body>
                </Content>
            </Layout>
        );
    }
}

export default Workers;


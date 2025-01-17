import React, { Component, Fragment } from "react";
import { Link } from "react-router-dom";
import { Table, Select, notification, Popconfirm, Row, Button, DatePicker, Space } from 'antd';
import { geekblue, lime, red, grey, orange } from '@ant-design/colors';
import axios from 'axios';
import styled from 'styled-components';

import {
  CheckCircleOutlined,
  SyncOutlined,
  CloseSquareOutlined,
  ClockCircleOutlined,
  MinusOutlined,
  WarningOutlined,
  QuestionCircleOutlined,
  LeftOutlined,
  DoubleRightOutlined,
} from '@ant-design/icons';
import { Token, TokenOverview, TokensRow, TokenState } from "../../types/Token";
import { datetime } from "../../types/common";
import { Moment } from "moment";
import { Task } from "../../types/Task";


const HeaderCell = styled.td`
    writing-mode: vertical-rl;
`;

const TCell = styled.td`
    border-bottom: 1px solid #ddd;
    padding-right: 15px;
`;

const TRow = styled.tr`
    padding-bottom: 15px;
    transition: background 0.3s;
    &:hover {
        > td {
            background-color: #f8f8f8;
        }
    }
`;


function iconForState(task: TokenState) {
    if (task === undefined) {
        return <MinusOutlined style={{color: grey[0]}} />;
    }

    let state = task.state;

    if (state == 'active') {
        return <SyncOutlined style={{color: grey[5]}}/>;
    } else if (state == 'running') {
        return <SyncOutlined spin style={{color: geekblue[5]}}/>;
    } else if (state == 'waiting') {
        return <ClockCircleOutlined style={{color: grey[5]}}/>;
    } else if (state == 'success') {
        return <CheckCircleOutlined style={{color: lime[5]}}/>;
    } else if (state == 'failure') {
        return <CloseSquareOutlined style={{color: red[5]}}/>;
    } else if (state == 'error') {
        return <WarningOutlined style={{color: orange[5]}}/>;
    } else {
        return 'invalid state?';
    }
}

async function activateToken(trigger_datetime: string, task_id: string) {
    await axios.put(`/api/tasks/${task_id}/tokens/${trigger_datetime}`, {});
    notification.success({
        message: 'Task Activated',
        description: 'The task has been activated and will run shortly.',
        placement: 'bottomRight',
    })
}

function makeCell(task: string, tok: TokensRow) {
    let this_task = tok.task_states[task];

    return (
        <TCell key={task}>
            <Popconfirm
                key="1"
                title={'Activate this task?'}
                okText={'Confirm'}
                cancelText={'Cancel'}
                okButtonProps={{size: 'middle'}}
                cancelButtonProps={{size: 'middle'}}
                onConfirm={() => activateToken(tok.trigger_datetime, this_task.task_id)}
                icon={<QuestionCircleOutlined style={{ color: geekblue[5] }}/>}
            >
                {iconForState(this_task)}    
            </Popconfirm>
        </TCell>
    );
}

function parseData(job_id: string, data: TokenOverview) {
    let {tasks, tokens} = data;


    let columns = <tr>{
        [
            <td key="trigger_datetime">Trigger Datetime</td>
        ].concat(tasks.map(t => (
            <HeaderCell key={t}>{t}</HeaderCell>
        )))
    }</tr>;


    let rows = tokens.map(tok => (
        <TRow key={tok.trigger_datetime}>{
            [
                <TCell key="trigger_datetime">
                    <Link to={`/jobs/${job_id}/tokens/${tok.trigger_datetime}`}>
                        {tok.trigger_datetime}
                    </Link>
                </TCell>
            ].concat(tasks.map(task => (
                makeCell(task, tok)
            )))
        }</TRow>
    ));


    return { columns, rows };
}

type TaskGridProps = {
    id: string;
};
type TaskGridState = {
    data: TokenOverview | null;
    limit: number;
    before: datetime | null;
    last?: datetime;
}

class TaskGrid extends Component<TaskGridProps, TaskGridState> {
    interval: NodeJS.Timeout;

    constructor(props: TaskGridProps) {
        super(props);

        this.state = {
            data: null,
            limit: 25,
            before: null,
        }
    }

    gotoCurrent() {
        this.setState({
            before: null,
        });
    }

    onDatePicked(date: Moment | null) {
        this.setState({
            before: date && date.toISOString()
        });
    }

    async fetchTokens() {
        const { id } = this.props;
        const { limit, before } = this.state;

        let params = {
            limit: limit,
            before: before,
        };

        let resp = await axios.get<TokenOverview>(`/api/jobs/${id}/tokens-overview`, {
                params: params
        });

        let last = resp.data.tokens[resp.data.tokens.length - 1].trigger_datetime;
        
        this.setState({
            data: resp.data,
            last: last,
        });
    }

    componentDidMount() {
        this.fetchTokens()

        // TODO - change back to 5s!
        // TODO - use a websocket to poll for token status changes
        this.interval = setInterval(() => this.fetchTokens(), 500);
    }

    componentWillUnmount() {
        clearInterval(this.interval);
    }

    render() {
        const { id }  = this.props;
        const { data } = this.state;

        if(!data) {
            return null;
        }

        const {rows, columns} = parseData(id, data);

        return (
            <Fragment>
                <Row>
                    <DatePicker onChange={(date) => this.onDatePicked(date)} />
                    <Space />
                    <Button onClick={() => this.gotoCurrent()} icon={<DoubleRightOutlined />}>
                        Latest
                    </Button>
                </Row>
                <Row>
                    <table>
                        <thead>
                            {columns}
                        </thead>
                        <tbody>
                            {rows}
                        </tbody>
                    </table>
                </Row>
            </Fragment>
        );
    }
}

export default TaskGrid;

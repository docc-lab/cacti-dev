const express = require('express');
const axios = require('axios');
const {json} = require("express");
require('dotenv').config();

const app = express();

app.use(express.json())

app.get('/', (req, res) => {
    res.send('<h1>Hello, Express.js Server!</h1>');
});

app.post('/spanquery', (req, res) => {
    const {
        start_year,
        start_month,
        start_day,
        start_hour,
        start_minute,
        end_year,
        end_month,
        end_day,
        end_hour,
        end_minute,
    } = req.body;

    console.log(req.body);

    res.status(200).send(`{ "query": "query queryTraces($condition: TraceQueryCondition) { data: queryBasicTraces(condition: $condition) { traces { key: segmentId endpointNames duration start isError traceIds } total } }", "variables": { "condition": { "queryDuration": { "start": "${start_year}-${start_month}-${start_day} ${start_hour}${start_minute}", "end": "${end_year}-${end_month}-${end_day} ${end_hour}${end_minute}", "step": "DAY"}, "traceState": "ALL", "paging": { "pageNum": 1, "pageSize": 10000, "needTotal": true }, "queryOrder": "BY_DURATION" } } }`);
});

app.post('/traces', (req, res) => {
    console.log(req.body);

    const singleQueryBuilder = (i) => {
        return `res${i}: queryTrace(traceId: $traceId${i}) { spans { traceId segmentId spanId parentSpanId serviceCode startTime endTime endpointName type peer component isError layer refs { traceId parentSegmentId parentSpanId type } } } `;
    }

    const multiQueryHeader = () => {
        let toReturn = 'query multiResult('
        const {
            traceIds
        } = req.body;

        for (let i in traceIds) {
            if (i !== 0) {
                toReturn += `, `
            }

            toReturn += `$traceId${i}: ID!`
        }

        toReturn += ') {'

        return toReturn;
    }

    const queryBuilder = () => {
        let toReturn = ' ';

        const {
            traceIds
        } = req.body;

        for (let i in traceIds) {
            toReturn += singleQueryBuilder(i);
        }

        return (multiQueryHeader() + toReturn + '}');
    }

    const variableBuilder = () => {
        const toReturn = {};

        const {
            traceIds
        } = req.body;

        for (let i in traceIds) {
            toReturn[`traceId${i}`] = traceIds[i];
        }

        return toReturn;
    }

    console.log(queryBuilder());
    console.log(variableBuilder());

    console.log(JSON.stringify({
        query: queryBuilder(),
        variables: variableBuilder(),
    }));

    axios.post(`http://localhost:${process.env.SKYWALKING_PORT}/graphql`, {
        query: queryBuilder(),
        variables: variableBuilder(),
    }, {
        headers: {
            "Content-Type": "application/json"
        }
    }).then((resp) => {
        console.log(resp.data);

        const { data } = resp.data;

        let toReturn = [];

        for (let result of Object.values(data)) {
            // toReturn.push(JSON.parse(data[key]));
            let trVal = {}
            for (let key in result) {
                if (key === 'spans') {
                    let trvSpans = [];
                    for (let span of result['spans']) {
                        let toPush = {};
                        for (let attr in span) {
                            if (attr === 'type') {
                                toPush['spanType'] = span['type'];
                            } else if (attr === 'refs') {
                                let refs = []
                                for (let ref of span['refs']) {
                                    let newRef = { ...ref };
                                    if (newRef.hasOwnProperty('type')) {
                                        newRef['refType'] = newRef['type'];
                                        delete newRef['type'];
                                    }
                                    refs.push(newRef)
                                }
                                toPush['refs'] = refs;
                            } else {
                                toPush[attr] = span[attr];
                            }
                        }
                        trvSpans.push(toPush);
                    }
                    trVal['spans'] = trvSpans;
                } else {
                    trVal[key] = result[key];
                }
            }
            toReturn.push(trVal);
        }

        res.status(200).json({
            success: true,
            data: toReturn,
            message: '',
        });
    }).catch((err) => {
        console.log(err);
        console.error(err);
        res.status(400).json({
            success: false,
            data: [],
            message: err.toString(),
        });
    })
});

const port = process.env.PORT || 3000; // You can use environment variables for port configuration
app.listen(port, () => {
    console.log(`Server is running on port ${port}`);
});










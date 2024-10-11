const express = require('express');
const axios = require('axios');
const {json} = require("express");
require('dotenv').config();

const app = express();

app.use(express.json())

app.get('/', (req, res) => {
    res.send('<h1>Hello, Express.js Server!</h1>');
});

app.post('/traces', (req, res) => {
    console.log(req.body);

    const singleQueryBuilder = (i) => {
        return `res{i}: queryTrace(traceId: $traceId${i}) { spans { traceId segmentId spanId parentSpanId serviceCode startTime endTime endpointName type peer component isError layer } } `;
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

    axios.post(`http://localhost:${process.env.SKYWALKING_PORT}/graphql`, {
        query: queryBuilder(),
        variables: variableBuilder(),
    }, {
        headers: {
            "Content-Type": "application/json"
        }
    }).then((resp) => {
        const { data } = resp.data;

        let toReturn = [];

        for (let key in data) {
            toReturn.push(JSON.parse(data[key]));
        }

        res.json({
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










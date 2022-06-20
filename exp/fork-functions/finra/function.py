import argparse
import json

from util import *
import yfinance as yf
import os
import sys
import time
import mmap
from agileutil.rpc.client import TcpRpcClient
from agileutil.rpc.server import rpc, TcpRpcServer
import asyncio

sys.path.append("../../common")  # include outer path

req = {"body": {"portfolioType": "S&P", "portfolio": "1234"}}
parser = argparse.ArgumentParser()
parser.add_argument("-port", type=int, default=8080, help="rpc server port")
args = parser.parse_args()
port = args.port


def public_data(event):
    """
    Fetch public market data from yahoo finance api
    :return:
    """
    startTime = 1000 * time.time()
    externalServicesTime = 0
    portfolioType = event['body']['portfolioType']

    tickers_for_portfolio_types = {'S&P': ['GOOG', 'AMZN', 'MSFT', 'SVFAU']}
    tickers = tickers_for_portfolio_types[portfolioType]

    prices = {}
    extra_data = {}
    item_cnt = 0
    for ticker in tickers:
        tickerObj = yf.Ticker(ticker)
        # Get last closing price
        tickTime = 1000 * time.time()
        data = tickerObj.history(period="max")
        externalServicesTime += 1000 * time.time() - tickTime
        prices[ticker] = data['Close'].unique()[0]
        item_cnt += data.count()
        extra_data[ticker] = data
    # prices = {'GOOG': 1732.38, 'AMZN': 3185.27, 'MSFT': 221.02}
    # print("item count", item_cnt)
    response = {'statusCode': 200,
                'body': {'marketData': prices, 'whole_set': extra_data}}

    endTime = 1000 * time.time()
    return timestamp(response, event, startTime, endTime, externalServicesTime)


def checkMarginBalance(portfolioData, marketData, portfolio):
    marginAccountBalance = {
        "1234": 4500
    }[portfolio]

    portfolioMarketValue = 0
    for trade in portfolioData:
        security = trade['Security']
        qty = trade['LastQty']
        portfolioMarketValue += qty * marketData[security]

    # Maintenance Margin should be atleast 25% of market value for "long" securities
    # https://www.finra.org/rules-guidance/rulebooks/finra-rules/4210#the-rule
    result = False
    if marginAccountBalance >= 0.25 * portfolioMarketValue:
        result = True

    return result


def bargin_balance(events):
    startTime = 1000 * time.time()
    marketData = {}
    validFormat = True

    for event in events:
        body = event['body']
        if 'marketData' in body:
            marketData = body['marketData']
        elif 'valid' in body:
            portfolio = event['body']['portfolio']
            validFormat = validFormat and body['valid']

    portfolioData = portfolios[portfolio]
    marginSatisfied = checkMarginBalance(portfolioData, marketData, portfolio)

    response = {'statusCode': 200,
                'body': {'validFormat': validFormat, 'marginSatisfied': marginSatisfied}}

    endTime = 1000 * time.time()
    return agg_timestamp(response, events, startTime, endTime, 0)


events = []


@rpc
def handler():
    global events, cli
    events[0] = cli.call('private_data', req)
    res = bargin_balance(events)
    return res


events = [None, public_data(req)]
cli = TcpRpcClient(servers=['127.0.0.1:8090'])
server = TcpRpcServer('0.0.0.0', port)
server.serve()

import socket

import numpy as np
import pandas as pd

from util import *

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

req = {"body": {"portfolioType": "S&P", "portfolio": "1234"}}


def public_data(event):
    """
    Fetch public market data from yahoo finance api
    :return:
    """
    startTime = 1000 * time.time()
    externalServicesTime = 0
    portfolioType = event['body']['portfolioType']

    tickers_for_portfolio_types = {'S&P': ['GOOG', 'AMZN', 'MSFT', 'SVFAU', 'AB', 'ABC', 'ABCB']}
    # stocks_list = list(pd.read_csv("stocks.csv")['Symbol'])
    tickers = tickers_for_portfolio_types[portfolioType]

    prices = {}
    whole_set = pd.read_csv("yfinance.csv")
    for ticker in tickers:
        # Get last closing price
        tickTime = 1000 * time.time()
        # data = pd.read_csv("yfinance.csv")
        externalServicesTime += 1000 * time.time() - tickTime
        prices[ticker] = whole_set['Close'].unique()[0]

    # prices = {'GOOG': 1732.38, 'AMZN': 3185.27, 'MSFT': 221.02}
    response = {'statusCode': 200,
                'body': {'marketData': prices, 'whole_set': whole_set}}

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


def analyse_whole_set(whole_set):
    finance_columns = ['Open', 'High', 'Low', 'Close', 'Volume', 'Dividends', 'Stock Splits']
    avg_data = [np.average(whole_set[key]) for key in finance_columns]
    sum_data = [np.sum(whole_set[key]) for key in finance_columns]
    std_data = [np.std(whole_set[key]) for key in finance_columns]
    return avg_data, sum_data, std_data


def bargin_balance(events):
    startTime = 1000 * time.time()
    marketData = {}
    whole_set = {}
    validFormat = True

    for event in events:
        body = event['body']
        if 'marketData' in body:
            marketData = body['marketData']
            whole_set = body['whole_set']
        elif 'valid' in body:
            portfolio = event['body']['portfolio']
            validFormat = validFormat and body['valid']

    portfolioData = portfolios[portfolio]
    marginSatisfied = checkMarginBalance(portfolioData, marketData, portfolio)
    whole_data = analyse_whole_set(whole_set)
    response = {'statusCode': 200,
                'body': {'validFormat': validFormat,
                         'marginSatisfied': marginSatisfied,
                         'whole_data': whole_data
                         }}

    endTime = 1000 * time.time()
    return agg_timestamp(response, events, startTime, endTime, 0)


def private_data(event):
    startTime = 1000 * time.time()

    portfolio = event['body']['portfolio']

    data = portfolios[portfolio]

    valid = True

    for trade in data:
        side = trade['Side']
        # Tag ID: 552, Tag Name: Side, Valid values: 1,2,8
        if not (side == 1 or side == 2 or side == 8):
            valid = False
            break
    response = {'statusCode': 200, 'body': {'valid': valid, 'portfolio': portfolio}}
    endTime = 1000 * time.time()
    return timestamp(response, event, startTime, endTime, 0)


events = [private_data(req), public_data(req)]


@tick_execution_time
def handler(time):
    for _ in range(time):
        res = bargin_balance(events)

handler(2)

def bench():
    handler(1)


if __name__ == '__main__':
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("localhost", 6000))
    s.listen(1)
    conn, address = s.accept()
    data = conn.recv(1024).decode()

    bench()
    conn.sendall('Parent finish'.encode())  # send back
    conn.close()
    s.close()

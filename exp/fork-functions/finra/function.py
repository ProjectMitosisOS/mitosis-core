from util import *
import yfinance as yf
import os
import sys
import time
import mmap

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *


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
    print("item count", item_cnt)
    response = {'statusCode': 200,
                'body': {'marketData': prices, 'whole_set': extra_data}}

    endTime = 1000 * time.time()
    return timestamp(response, event, startTime, endTime, externalServicesTime)


def init():
    pass


# @tick_execution_time
def handler():
    print("hello world")


# @mitosis_bench
def bench():
    handler()


if __name__ == '__main__':
    init()
    bench()

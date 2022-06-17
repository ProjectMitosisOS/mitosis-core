import ctypes
import sys
import time

def timestamp(response, event, startTime, endTime, externalServicesTime):
    stampBegin = 1000*time.time()
    prior = event['duration'] if 'duration' in event else 0
    priorMemSet = event['memsetTime'] if 'memsetTime' in event else 0
    priorServiceTime = event['externalServicesTime'] if 'externalServicesTime' in event else 0
    response['duration']     = prior + endTime - startTime
    response['workflowEndTime'] = endTime
    response['workflowStartTime'] = event['workflowStartTime'] if 'workflowStartTime' in event else startTime
    priorCost = event['timeStampCost'] if 'timeStampCost' in event else 0
    response['externalServicesTime'] = priorServiceTime + externalServicesTime
    response['memsetTime'] = priorMemSet
    response['timeStampCost'] = priorCost - (stampBegin-1000*time.time())
    return response

def agg_timestamp(response, events, startTime, endTime, externalServicesTime):
    stampBegin = 1000*time.time()
    prior = 0
    priorCost = 0
    priorServiceTime = 0
    workflowStartTime = startTime
    priorEndTime = 0
    externalServiceTimes = []

    for event in events:
        if 'workflowEndTime' in event and event['workflowEndTime'] > priorEndTime:
            priorEndTime = event['workflowEndTime']
            priorCost = event['timeStampCost']
        if 'workflowStartTime' in event and event['workflowStartTime'] < workflowStartTime:
            workflowStartTime = event['workflowStartTime']
        if 'externalServicesTime' in event and event['externalServicesTime'] > 0:
            externalServiceTimes.append(event['externalServicesTime'])

    priorServiceTime = max(externalServiceTimes) if len(externalServiceTimes) else 0

    #NOTE: This works only if the parallel step is the first step in the workflow
    prior = priorEndTime - workflowStartTime
    response['duration']     = prior + endTime - startTime
    response['workflowEndTime'] = endTime
    response['workflowStartTime'] = workflowStartTime
    response['externalServicesTime'] = priorServiceTime + externalServicesTime
    response['memsetTime'] = 0

    #Obscure code, doing to time.time() at the end of fn
    response['timeStampCost'] = priorCost - (stampBegin-1000*time.time())
    return response

def clear_output(value):
    startTime = 1000*time.time()
    vt = type(value)
    if vt == str:
        bufSize = len(value) + 1
        offset  = sys.getsizeof(value) - bufSize
        ctypes.memset(id(value)+offset, 0, bufSize)
    elif vt == int:
        bufSize = sys.getsizeof(value)-24
        # ctypes.memset(id(value)+24, 0, bufSize)
    elif vt == float:
        bufSize = sys.getsizeof(value)-16
        ctypes.memset(id(value)+16, 0, bufSize)
    elif vt == dict:
        for k,v in value.items():
            clear_output(v)
    elif vt == list:
        for k in value:
            clear_output(k)

    return (1000*time.time()-startTime)

def copy_output(v):
    output = {}
    vt = type(v)
    if vt == str:
        nv = v + '0'
        nv = nv[:-1]
    elif vt == int:
        nv = v + 0
    elif vt == float:
        nv = v + 0.0
    elif vt == dict:
        nv = {}
        for k, s in v.items():
            nv[k] = copy_output(s)
    elif vt == list:
        nv = []
        for k in v:
            nv.append(copy_output(k))
    return nv

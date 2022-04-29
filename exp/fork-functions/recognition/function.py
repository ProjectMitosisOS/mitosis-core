import json
import os
import sys

from PIL import Image

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'
import torch
from torchvision import transforms
from torchvision.models import resnet50

tmp = '/dev/shm/'

model = resnet50(pretrained=False)
# model.load_state_dict(torch.load(model_path))
model.eval()

input_image = Image.open("test.jpg")

@tick_execution_time
def handler():
    preprocess = transforms.Compose([
        transforms.Resize(256),
        transforms.CenterCrop(224),
        transforms.ToTensor(),
        transforms.Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225]),
    ])
    input_tensor = preprocess(input_image)
    input_batch = input_tensor.unsqueeze(0)  # create a mini-batch as expected by the model
    output = model(input_batch)
    _, index = torch.max(output, 1)
    # The output has unnormalized scores. To get probabilities, you can run a softmax on it.
    prob = torch.nn.functional.softmax(output[0], dim=0)
    _, indices = torch.sort(output, descending=True)


@mitosis_bench
def bench():
    handler()


if __name__ == '__main__':
    bench()

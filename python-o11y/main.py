import random
import time
from fastapi import FastAPI

app = FastAPI()


@app.get("/colour")
def pick_random_colour():
    available_colours = ["red", "yellow", "green", "blue", "pink"]
    chosen_colour = random.choice(available_colours)
    long_sleeping_process(1)
    return {"colour": chosen_colour}


def long_sleeping_process(time_to_sleep: int):
    time.sleep(time_to_sleep)

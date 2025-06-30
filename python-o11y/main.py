import random
import time
from fastapi import FastAPI
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI()

FastAPIInstrumentor.instrument_app(app)


@app.get("/colour")
def pick_random_colour():
    available_colours = ["red", "yellow", "green", "blue", "pink"]
    chosen_colour = random.choice(available_colours)
    logger.info(f"Chosen colour: {chosen_colour}")
    long_sleeping_process(1)
    return {"colour": chosen_colour}


def long_sleeping_process(time_to_sleep: int):
    logger.info(f"Going to sleep for {time_to_sleep} seconds")
    time.sleep(time_to_sleep)

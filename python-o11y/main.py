import logging
import random
import time

from fastapi import FastAPI
from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor
from opentelemetry.instrumentation.logging import LoggingInstrumentor
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.semconv.attributes.service_attributes import SERVICE_NAME

LoggingInstrumentor().instrument(set_logging_format=True)

resource = Resource.create({SERVICE_NAME: "python-o11y-service"})

provider = TracerProvider(resource=resource)

otlp_exporter = OTLPSpanExporter(insecure=True)
span_processor = BatchSpanProcessor(otlp_exporter)
provider.add_span_processor(span_processor)

trace.set_tracer_provider(provider)
tracer = trace.get_tracer(__name__)

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI()
logger.error("Lol")

FastAPIInstrumentor.instrument_app(app)


@app.get("/colour")
def pick_random_colour():
    with tracer.start_as_current_span("get_random_colour"):
        available_colours = ["red", "yellow", "green", "blue", "pink"]
        chosen_colour = random.choice(available_colours)

        logger.info(f"Chosen colour: {chosen_colour}")
        long_sleeping_process(1)
        return {"colour": chosen_colour}


@app.get("/colours")
def pick_two_random_colours():
    with tracer.start_as_current_span("get_random_colours"):
        first_colour = pick_random_colour()["colour"]
        second_colour = pick_random_colour()["colour"]
        logger.info(f"Chosen two colours: {first_colour} and {second_colour}")
        return {"colours": [first_colour, second_colour]}


def long_sleeping_process(time_to_sleep: int):
    with tracer.start_as_current_span("long_sleeping_process"):
        logger.info(f"Going to sleep for {time_to_sleep} seconds")
        time.sleep(time_to_sleep)


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, port="8000")

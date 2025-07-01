import logging
import random
import time

from fastapi import FastAPI
from opentelemetry import trace
from opentelemetry._logs import set_logger_provider
from opentelemetry.exporter.otlp.proto.grpc._log_exporter import OTLPLogExporter
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor
from opentelemetry.instrumentation.logging import LoggingInstrumentor
from opentelemetry.sdk._logs import LoggerProvider, LoggingHandler
from opentelemetry.sdk._logs.export import BatchLogRecordProcessor
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.semconv.attributes.service_attributes import SERVICE_NAME

resource = Resource.create({SERVICE_NAME: "python-o11y-service"})

tracer_provider = TracerProvider(resource=resource)
otlp_span_exporter = OTLPSpanExporter(insecure=True)
span_processor = BatchSpanProcessor(otlp_span_exporter)
tracer_provider.add_span_processor(span_processor)
trace.set_tracer_provider(tracer_provider)
tracer = trace.get_tracer(__name__)

logger_provider = LoggerProvider(resource=resource)
otlp_log_exporter = OTLPLogExporter(insecure=True)
log_processor = BatchLogRecordProcessor(otlp_log_exporter)
logger_provider.add_log_record_processor(log_processor)
handler = LoggingHandler(level=logging.NOTSET, logger_provider=logger_provider)
set_logger_provider(logger_provider)

logging.getLogger().setLevel(logging.NOTSET)
logging.getLogger().addHandler(handler)

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI()
logger.error("Lol")

LoggingInstrumentor().instrument(set_logging_format=True)
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

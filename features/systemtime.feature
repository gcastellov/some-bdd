Feature: System time

    Scenario: System time is retrieved from public API
        Given request is not authenticated
        When system time is requested
        Then gets successful response as json
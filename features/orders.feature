Feature: Current orders

    Scenario: Get all current open orders
        Given request is authenticated
        When all current open orders are requested
        Then gets successful response as json
        And response contains error list as empty
        And response contains order list as empty
Feature: Asset information

    Scenario: Asset information pair is retrieved from public API
        Given request is not authenticated
        When asset pair information is requested for XBT and USD
        Then gets successful response as json
        And response contains error empty
        And response only contains asset pair information XXBTZUSD
        And asset pair information for XBT and USD as XXBTZUSD is as expected
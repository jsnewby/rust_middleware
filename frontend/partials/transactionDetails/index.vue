<template>
  <div class="transaction-details">
    <AppPanel>
      <AppTable details>
        <AppTableBody>
          <AppTableRow extend>
            <AppTableCell extend>
              <TxlistItem :data="data" />
            </AppTableCell>
          </AppTableRow>
          <AppTableRow extend>
            <AppTableCell extend>
              <AppDefinition
                type="list"
                title="Hash"
              >
                <FormatAddress
                  :value="data.hash"
                  length="full"
                  icon
                />
              </AppDefinition>
            </AppTableCell>
          </AppTableRow>
          <AppTableRow
            v-if="data.arguments"
            extend
          >
            <AppTableCell extend>
              <AppDefinition
                type="list"
                title="Arguments"
              >
                {{ data.arguments.arguments }}
              </AppDefinition>
            </AppTableCell>
          </AppTableRow>
          <AppTableRow
            v-if="data.arguments"
            extend
          >
            <AppTableCell extend>
              <AppDefinition
                type="list"
                title="Method"
              >
                {{ data.arguments.function }}
              </AppDefinition>
            </AppTableCell>
          </AppTableRow>
          <AppTableRow
            v-if="data.callinfo && data.callinfo.return_type"
            extend
          >
            <AppTableCell extend>
              <AppDefinition
                type="list"
                title="Return Type"
              >
                {{ data.callinfo.return_type }}
              </AppDefinition>
            </AppTableCell>
          </AppTableRow>
          <AppTableRow
            v-if="data.callinfo && data.callinfo.return_value"
            extend
          >
            <AppTableCell extend>
              <AppDefinition
                type="list"
                title="Return Value"
              >
                {{ data.callinfo.return_value }}
              </AppDefinition>
            </AppTableCell>
          </AppTableRow>
          <AppTableRow
            v-if="data.result"
            extend
          >
            <AppTableCell extend>
              <AppDefinition
                type="list"
                title="Return Value (Decoded)"
              >
                {{ data.result }}
              </AppDefinition>
            </AppTableCell>
          </AppTableRow>
          <AppTableRow
            v-if="data.callinfo && data.callinfo.log"
            extend
          >
            <AppTableCell extend>
              <AppDefinition
                type="list"
                title="log"
              >
                {{ data.callinfo.log }}
              </AppDefinition>
            </AppTableCell>
          </AppTableRow>
          <AppTableRow
            v-if="data.ttl"
            extend
          >
            <AppTableCell extend>
              <AppDefinition
                type="list"
                title="Ttl"
              >
                {{ data.tx.ttl }}
              </AppDefinition>
            </AppTableCell>
          </AppTableRow>
          <AppTableRow
            v-if="data.nonce"
            extend
          >
            <AppTableCell extend>
              <AppDefinition
                type="list"
                title="Nonce"
              >
                {{ data.tx.nonce }}
              </AppDefinition>
            </AppTableCell>
          </AppTableRow>
          <AppTableRow
            v-if="data.signatures"
            extend
          >
            <AppTableCell extend>
              <AppDefinition
                type="list"
                title="Signatures"
              >
                <FormatAddress
                  :value="data.signatures[0]"
                  length="full"
                />
              </AppDefinition>
            </AppTableCell>
          </AppTableRow>
          <AppTableRow
            v-if="data.tx.tx"
            extend
          >
            <AppTableCell extend>
              <AppDefinition
                type="list"
                title="Embedded Tx"
              >
                {{ data.tx.tx }}
              </AppDefinition>
            </AppTableCell>
          </AppTableRow>
          <AppTableRow extend>
            <AppTableCell extend>
              <AppDefinition
                type="list"
                title="Version"
              >
                {{ data.tx.version }}
              </AppDefinition>
            </AppTableCell>
          </AppTableRow>
        </AppTableBody>
      </AppTable>
    </AppPanel>
  </div>
</template>

<script>
import AppTable from '../../components/appTable'
import AppTableRow from '../../components/appTableRow'
import AppTableCell from '../../components/appTableCell'
import AppTableBody from '../../components/appTableBody'
import AppDefinition from '../../components/appDefinition'
import AppPanel from '../../components/appPanel'
import FormatAddress from '../../components/formatAddress'
import TxlistItem from '../../partials/transactions/txListItem'

export default {
  name: 'TransactionDetails',
  components: {
    AppTable,
    AppTableRow,
    AppTableCell,
    AppTableBody,
    AppDefinition,
    AppPanel,
    // TimeStamp,
    FormatAddress,
    TxlistItem
  },
  props: {
    data: {
      type: Object,
      default: undefined
    }
  }
}
</script>

<style scoped lang="scss">
  .transaction-details {
    margin-bottom: 2rem;
  }
  .block-height-wrapper {
    display: flex;
    flex-direction: column;
    @media (min-width: 550px) {
      flex-direction: row;
      align-items: center;
    }
  }
</style>

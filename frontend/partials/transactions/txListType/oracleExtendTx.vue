<template>
  <div class="transaction">
    <div class="transaction-main-info">
      <div class="transaction-main-info-inner">
        <div class="transaction-label">
          <LabelType
            :title="transaction.tx.type.replace(/([A-Z])/g, ' $1')"
            fill="red"
          />
        </div>
        <AppDefinition
          v-if="transaction.tx.time"
          title="Age"
        >
          <Age :time="transaction.tx.time" />
        </AppDefinition>
      </div>
      <div class="transaction-main-info-inner accounts">
        <AccountGroup>
          <Account
            v-if="transaction.tx.account_id"
            :value="transaction.tx.account_id"
            title="account"
            icon
          />
          <Account
            v-if="transaction.tx.sender_id"
            :value="transaction.tx.sender_id"
            title="Sender"
            icon
          />
          <Account
            v-if="transaction.tx.recipient_id"
            :value="transaction.tx.recipient_id"
            title="recipient"
            icon
          />
        </AccountGroup>
      </div>
    </div>
    <div class="transaction-type-info">
      <div class="transaction-type-info-item" />
      <div class="transaction-type-info-item">
        <AppDefinition
          v-if="transaction.tx.fee"
          title="tx fee"
        >
          <FormatAeUnit
            :value="transaction.tx.fee"
          />
        </AppDefinition>
        <AppDefinition
          v-if="transaction.tx.cost"
          title="tx cost"
        >
          <FormatAeUnit
            :value="transaction.tx.cost"
          />
        </AppDefinition>
      </div>
    </div>
  </div>
</template>
<script>
import AppDefinition from '../../../components/appDefinition'
import FormatAeUnit from '../../../components/formatAeUnit'
import AccountGroup from '../../../components/accountGroup'
import Account from '../../../components/account'
import Age from '../../../components/age'
import LabelType from '../../../components/labelType'

export default {
  name: 'OracleExtendTx',
  components: {
    LabelType,
    AppDefinition,
    FormatAeUnit,
    AccountGroup,
    Account,
    Age
  },
  props: {
    transaction: {
      type: Object,
      required: true
    }
  }
}
</script>

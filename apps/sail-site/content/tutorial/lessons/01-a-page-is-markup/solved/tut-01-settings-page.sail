<!--
  Lesson 01 solved — a settings screen with zero JavaScript.

  Every visual decision here is a tag, a layout attribute, or a component
  prop. There is no controller file for this lesson: structure comes from
  the layout keywords, appearance from the tone/variant/size contracts.
-->
<scope>
  style title = { type(heading) }
  style muted = { fg(muted).type(body-sm) }
  style hint = { fg(muted) }
  style rowLabel = { type(body-sm) }
  type NotifyRowProps = { label: string; detail: string; enabled: boolean }
</scope>

<partial notify-row props: NotifyRowProps>
  <row align="center" justify="between" gap="3">
    <column gap="1">
      <strong style(rowLabel)>{label}</strong>
      <small style(hint)>{detail}</small>
    </column>
    <switch checked={enabled} tone="primary"></switch>
  </row>
</partial>

<column gap="5">
  <row align="center" justify="between" gap="4">
    <column gap="1">
      <h2 style(title)>Workspace settings</h2>
      <p style(muted)>Profile, notifications, and the scary stuff.</p>
    </column>
    <button variant="solid" tone="primary">Save changes</button>
  </row>

  <grid cols="2" gap="4">
    <card padding="lg">
      <column gap="4">
        <row gap="2" align="center">
          <h3>Profile</h3>
          <badge tone="success" variant="soft" size="sm">verified</badge>
        </row>
        <row gap="3" align="center">
          <avatar initials="AV" tone="primary" size="lg"></avatar>
          <column gap="1">
            <strong>Avery Vale</strong>
            <small style(hint)>avery@example.dev</small>
          </column>
        </row>
        <separator></separator>
        <row gap="2" justify="end">
          <button variant="ghost" size="sm">Change photo</button>
          <button variant="outline" tone="primary" size="sm">Edit profile</button>
        </row>
      </column>
    </card>

    <card padding="lg">
      <column gap="4">
        <h3>Notifications</h3>
        <notify-row
          label="Weekly digest"
          detail="One summary email, Monday morning."
          enabled={true}
        />
        <notify-row
          label="Mentions"
          detail="When someone @-mentions you."
          enabled={true}
        />
        <notify-row
          label="Marketing"
          detail="Product news and tips."
          enabled={false}
        />
      </column>
    </card>
  </grid>

  <card tone="destructive" variant="outline" padding="lg">
    <row align="center" justify="between" gap="4">
      <column gap="1">
        <h3>Danger zone</h3>
        <p style(muted)>Deleting the workspace removes every project in it.</p>
      </column>
      <button variant="outline" tone="destructive">Delete workspace</button>
    </row>
  </card>
</column>

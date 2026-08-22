<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style pageHead = { pad({ b: 4 }).border({ width: 1, color: border, edge: b }) }
  style title = { margin({ b: 2 }).fg(foreground).type({ size: 2xl, weight: bold }) }
  style lede = { fg(muted-foreground).type({ size: sm }) }
  style sectionH = { margin({ b: 4 }).fg(foreground).type({ size: lg, weight: semibold, family: mono }) }
  style card = { pad(4).border({ width: 1, color: border }).radius(lg).bg(surface) }
  style cardLabel = { fg(muted-foreground).type({ size: sm, weight: medium }) }
  style legendPrimary = { fg(primary) }
  style textSm = { type({ size: sm }) }
  style textXs = { type({ size: xs }) }
</scope>

<column gap="8" style(page)>
  <header style(pageHead)>
    <h1 style(title)>sail/charts</h1>
    <p style(lede)>Pack <strong>sail/charts</strong> — SVG-rendered chart primitives. Each card shows one component with representative prop/variant combinations.</p>
  </header>
  <section>
    <h2 style(sectionH)>sail-area-chart</h2>

    <row align="start">
      <column gap="3" style(card)>
        <h3 style(cardLabel)>single series, no grid</h3>
        <sail-area-chart
          series='[{"name":"Revenue","data":[12,28,18,42,35,60,55]}]'
          labels="Mon,Tue,Wed,Thu,Fri,Sat,Sun"
          label="revenue area chart">
        </sail-area-chart>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>two series + grid + points</h3>
        <sail-area-chart
          series='[{"name":"Downloads","data":[5,14,9,22,18,30,27],"tone":"primary"},{"name":"Uploads","data":[2,6,4,10,8,14,12],"tone":"success"}]'
          labels="Mon,Tue,Wed,Thu,Fri,Sat,Sun"
          grid
          points
          label="downloads vs uploads area chart">
          <span slot="legend-entry" style(legendPrimary)>Downloads</span>
          <span>Uploads</span>
        </sail-area-chart>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>loading state</h3>
        <sail-area-chart loading label="loading area chart"></sail-area-chart>
      </column>
    </row>
  </section>
  <section>
    <h2 style(sectionH)>sail-line-chart</h2>

    <row align="start">
      <column gap="3" style(card)>
        <h3 style(cardLabel)>single series</h3>
        <sail-line-chart
          series='[{"name":"Latency","data":[80,72,95,60,88,74,65]}]'
          labels="Mon,Tue,Wed,Thu,Fri,Sat,Sun"
          label="latency line chart">
        </sail-line-chart>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>two series + grid + points</h3>
        <sail-line-chart
          series='[{"name":"p50","data":[40,38,45,36,42,39,37],"tone":"primary"},{"name":"p99","data":[120,115,140,105,130,118,110],"tone":"destructive"}]'
          labels="Mon,Tue,Wed,Thu,Fri,Sat,Sun"
          grid
          points
          label="p50 vs p99 line chart">
        </sail-line-chart>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>loading state</h3>
        <sail-line-chart loading label="loading line chart"></sail-line-chart>
      </column>
    </row>
  </section>
  <section>
    <h2 style(sectionH)>sail-bar-chart</h2>

    <row align="start">
      <column gap="3" style(card)>
        <h3 style(cardLabel)>vertical · tone=primary</h3>
        <sail-bar-chart
          data="12,28,18,42,35,60,55"
          labels="Mon,Tue,Wed,Thu,Fri,Sat,Sun"
          tone="primary"
          orientation="vertical"
          label="weekly bar chart">
        </sail-bar-chart>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>vertical · tone=success</h3>
        <sail-bar-chart
          data="5,14,9,22,18,30,27"
          labels="Mon,Tue,Wed,Thu,Fri,Sat,Sun"
          tone="success"
          orientation="vertical"
          label="success bar chart">
        </sail-bar-chart>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>horizontal · tone=warning</h3>
        <sail-bar-chart
          data="30,55,40,70,25"
          labels="Alpha,Beta,Gamma,Delta,Epsilon"
          tone="warning"
          orientation="horizontal"
          label="horizontal bar chart">
        </sail-bar-chart>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>horizontal · tone=destructive</h3>
        <sail-bar-chart
          data="8,16,12,20,5"
          labels="Q1,Q2,Q3,Q4,Q5"
          tone="destructive"
          orientation="horizontal"
          label="destructive bar chart">
        </sail-bar-chart>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>loading state</h3>
        <sail-bar-chart loading label="loading bar chart"></sail-bar-chart>
      </column>
    </row>
  </section>
  <section>
    <h2 style(sectionH)>sail-donut</h2>

    <row align="start">
      <column gap="3" style(card)>
        <h3 style(cardLabel)>default size, center label</h3>
        <sail-donut
          data='[{"label":"TypeScript","value":58,"tone":"primary"},{"label":"Rust","value":30,"tone":"success"},{"label":"Other","value":12,"tone":"muted"}]'
          label="language breakdown donut">
          <span>58%</span>
        </sail-donut>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>larger size, thicker ring</h3>
        <sail-donut
          data='[{"label":"CPU","value":42,"tone":"destructive"},{"label":"Memory","value":35,"tone":"warning"},{"label":"Free","value":23,"tone":"muted"}]'
          size="160"
          thickness="24"
          label="resource usage donut">
          <span style(textSm)>Resources</span>
        </sail-donut>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>compact, thin ring</h3>
        <sail-donut
          data='[{"label":"Done","value":72,"tone":"success"},{"label":"Remaining","value":28,"tone":"muted"}]'
          size="80"
          thickness="8"
          label="progress donut">
          <span style(textXs)>72%</span>
        </sail-donut>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>loading state</h3>
        <sail-donut loading label="loading donut"></sail-donut>
      </column>
    </row>
  </section>
  <section>
    <h2 style(sectionH)>sail-gauge</h2>

    <row align="start">
      <column gap="3" style(card)>
        <h3 style(cardLabel)>tone=primary, 65%</h3>
        <sail-gauge
          value="65"
          min="0"
          max="100"
          tone="primary"
          label="cpu gauge">
          <span>65%</span>
        </sail-gauge>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>tone=success, 85%</h3>
        <sail-gauge
          value="85"
          min="0"
          max="100"
          tone="success"
          label="health gauge">
          <span>85%</span>
        </sail-gauge>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>tone=warning, 45%</h3>
        <sail-gauge
          value="45"
          min="0"
          max="100"
          tone="warning"
          label="memory gauge">
          <span>45%</span>
        </sail-gauge>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>tone=destructive, 90%</h3>
        <sail-gauge
          value="90"
          min="0"
          max="100"
          tone="destructive"
          label="disk gauge">
          <span>90%</span>
        </sail-gauge>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>threshold-driven color, high=70</h3>
        <sail-gauge
          value="55"
          min="0"
          max="100"
          low-threshold="30"
          high-threshold="70"
          size="160"
          thickness="16"
          label="threshold gauge">
          <span>55 / 100</span>
        </sail-gauge>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>loading state</h3>
        <sail-gauge loading label="loading gauge"></sail-gauge>
      </column>
    </row>
  </section>
  <section>
    <h2 style(sectionH)>sail-heatmap</h2>

    <row align="start">
      <column gap="3" style(card)>
        <h3 style(cardLabel)>tone=primary, 4×7 grid</h3>
        <sail-heatmap
          data="0,3,7,2,5,8,4,1,6,9,3,7,2,5,0,4,8,6,3,7,2,5,9,4,1,6,3,8"
          rows="Mon,Tue,Wed,Thu"
          cols="W1,W2,W3,W4,W5,W6,W7"
          tone="primary"
          cell-size="24"
          label="activity heatmap">
        </sail-heatmap>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>tone=success, compact cells</h3>
        <sail-heatmap
          data="0,2,5,8,3,6,9,1,4,7,2,5,8,3,6,9,1,4,7,2,5,8,3,6,9,1,4,7"
          rows="Mon,Tue,Wed,Thu"
          cols="W1,W2,W3,W4,W5,W6,W7"
          tone="success"
          cell-size="16"
          gap="2"
          label="success heatmap">
        </sail-heatmap>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>tone=warning</h3>
        <sail-heatmap
          data="1,3,6,2,8,4,9,0,5,7,3,6,2,8,4,9,0,5,7,3,6,2,8,4,9,0,5,7"
          rows="Mon,Tue,Wed,Thu"
          cols="W1,W2,W3,W4,W5,W6,W7"
          tone="warning"
          cell-size="20"
          label="warning heatmap">
        </sail-heatmap>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>loading state</h3>
        <sail-heatmap loading label="loading heatmap"></sail-heatmap>
      </column>
    </row>
  </section>
  <section>
    <h2 style(sectionH)>sail-sparkline</h2>

    <row align="start">
      <column gap="3" style(card)>
        <h3 style(cardLabel)>tone variants</h3>
        <grid cols="2">
          <span style(lede)>primary</span>
          <sail-sparkline data="4,7,3,9,5,11,8" tone="primary" width="120" height="32"></sail-sparkline>
          <span style(lede)>success</span>
          <sail-sparkline data="2,5,8,6,9,7,10" tone="success" width="120" height="32"></sail-sparkline>
          <span style(lede)>warning</span>
          <sail-sparkline data="9,6,8,4,7,5,6" tone="warning" width="120" height="32"></sail-sparkline>
          <span style(lede)>destructive</span>
          <sail-sparkline data="3,8,5,10,6,9,4" tone="destructive" width="120" height="32"></sail-sparkline>
          <span style(lede)>accent</span>
          <sail-sparkline data="6,4,9,3,8,5,7" tone="accent" width="120" height="32"></sail-sparkline>
          <span style(lede)>muted</span>
          <sail-sparkline data="5,7,4,8,6,9,3" tone="muted" width="120" height="32"></sail-sparkline>
        </grid>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>fill=true vs fill=false</h3>
        <grid cols="2">
          <span style(lede)>no fill</span>
          <sail-sparkline data="4,7,3,9,5,11,8" tone="primary" width="120" height="32"></sail-sparkline>
          <span style(lede)>fill</span>
          <sail-sparkline data="4,7,3,9,5,11,8" tone="primary" fill width="120" height="32"></sail-sparkline>
        </grid>
      </column>

      <column gap="3" style(card)>
        <h3 style(cardLabel)>loading state</h3>
        <sail-sparkline loading tone="primary" width="120" height="32"></sail-sparkline>
      </column>
    </row>
  </section>
</column>

import time

from locust import FastHttpUser as HttpUser, task


class BaseUser(HttpUser):
    def get(self, flow, weight, cost, delay=0):
        t0 = time.time()
        response = self.client.get(f"/test/{flow}/{weight}/{cost}")
        actual_cost = (time.time() - t0) * 1000
        if actual_cost < cost:
            time.sleep((cost - actual_cost) / 1000)
        if delay:
            time.sleep(delay / 1000)


class User(BaseUser):
    @task
    def flow_1(self):
        self.get(flow=1, weight=1, cost=50)

    @task
    def flow_2(self):
        self.get(flow=2, weight=2, cost=50)


class User2(BaseUser):
    @task(1)
    def flow_1(self):
        self.get(flow=1, weight=1, cost=50)

    @task(2)
    def flow_2(self):
        self.get(flow=2, weight=2, cost=50)



class User3(BaseUser):
    @task(1)
    def flow_1(self):
        self.get(flow=1, weight=1, cost=50, delay=1000)

    @task(10)
    def flow_2(self):
        self.get(flow=2, weight=4, cost=50)


class User4(BaseUser):
    @task(4)
    def flow_1(self):
        self.get(flow=1, weight=1, cost=50)

    @task(10)
    def flow_2(self):
        self.get(flow=2, weight=1, cost=50)


class User5(BaseUser):
    @task(1)
    def flow_1(self):
        self.get(flow=1, weight=1, cost=50)

    @task(1)
    def flow_2(self):
        self.get(flow=2, weight=1, cost=50)

    @task(1)
    def flow_3(self):
        self.get(flow=3, weight=1, cost=50)


class User6(BaseUser):
    @task(1)
    def flow_1(self):
        self.get(flow=1, weight=1, cost=50)

    @task(1)
    def flow_2(self):
        self.get(flow=2, weight=1, cost=50)

    @task(1)
    def flow_3(self):
        self.get(flow=3, weight=9, cost=50)
